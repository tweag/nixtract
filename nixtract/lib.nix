{ nixpkgs ? builtins.getFlake "nixpkgs" }:

rec {
  /* Utility function to enumerate a list
    Type: [a] -> [(int, a)]
  
    Example:
    enumerate [ "a" "b" ]
    => [ { index = 0; value = "a"; } { index = 1; value = "b"; } ]
  */
  enumerate = lst: map (zippedElem: { index = zippedElem.fst; value = zippedElem.snd; }) (nixpkgs.lib.zipLists (nixpkgs.lib.range 0 (builtins.length lst)) lst);

  /* To camel case to snake case
    Type: string -> string
  */
  toSnakeCase = string: nixpkgs.lib.concatStrings (map (s: if builtins.isString s then s else "_" + nixpkgs.lib.toLower (builtins.elemAt s 0)) (builtins.split "([A-Z])" string));

  /* A modified version of nixpkgs lib.attrsets.collect to go inside lists as well
  */
  collect =
    pred:
    v:
    if pred v then
      [ v ]
    else if nixpkgs.lib.isAttrs v then
      nixpkgs.lib.concatMap (collect pred) (nixpkgs.lib.attrValues v)
    else if nixpkgs.lib.isList v then
      nixpkgs.lib.concatMap (collect pred) v
    else
      [ ];

  /* Packages in a flake are usually a flat attribute set in outputs, but legacy systems use `legacyPackages`
  */
  getFlakePkgs = flake: targetSystem: flake.outputs.packages.${targetSystem} or flake.outputs.defaultPackage.${targetSystem} or flake.outputs.legacyPackages.${targetSystem} or { };


  /* Follow "attribute path" (split by dot) to access value in tree of nested attribute sets and lists
    Type: (attrs | list) -> str -> any

    Examples:
    getValueAtPath {a = { b = 1; c = [ 2 ]; }; } "a.b"
    => 1

    getValueAtPath {a = { b = 1; c = [ 2 ]; }; } "a.c.0"
    => 2
  */
  getValueAtPath =
    collection: attributePath:
    let recurse =
      collection: pathList:
      let
        x = builtins.head pathList;
        value =
          if nixpkgs.lib.isAttrs collection
          then collection.${x}
          else
            if nixpkgs.lib.isList collection
            then
              let index = nixpkgs.lib.toIntBase10 x;
              in builtins.elemAt collection index
            else builtins.throw "Trying to follow path in neither an attribute set nor a list";
      in
      if builtins.length pathList > 1 then
        let
          # need to skip an item, see `builtins.split` doc
          xs = builtins.tail (builtins.tail pathList);
        in
        recurse value xs
      else value;
    in recurse collection (builtins.split "\\." attributePath);

  /* Utility function for safe evaluation of any value, null if evaluation fails
  */
  safeEval = v: (builtins.tryEval v).value or null;

  /* Utility specific to nixpkgs, as nixpkgs prevents computing some fields if meta.platforms does not contain the target system
    Args:
    targetSystem: (string) target system
    f: (derivation -> a) function to apply to derivation
    drv: derivation
  */
  safePlatformDrvEval =
    targetSystem: f: drv:
    if !(builtins.elem targetSystem (drv.meta.platforms or [ targetSystem ]))
    then null
    else safeEval (f drv);
}
