/* Find the attribute paths to derivations directly available in a flake.

Results are written to stderr, prefixed by `trace: `, as a JSON dict with key "foundDrvs".

Args (as environment variables):
    TARGET_FLAKE_REF: flake reference to evaluate
    TARGET_SYSTEM: system to evaluate

Example:
TARGET_FLAKE_REF="nixpkgs" TARGET_SYSTEM="x86_64-linux" nix eval --json --file ./find-attribute-paths.nix
*/

let
  nixpkgs = builtins.getFlake "nixpkgs";
  lib = import ./lib.nix { inherit nixpkgs; };

  # Arguments have to be taken from environment when using `nix` command
  targetFlakeRef = builtins.getEnv "TARGET_FLAKE_REF";
  targetSystem = builtins.getEnv "TARGET_SYSTEM";

  # Get pkgs
  targetFlake = builtins.getFlake targetFlakeRef;
  targetFlakePkgs = lib.getFlakePkgs targetFlake targetSystem;

  # Describe briefly found derivation
  describeDrv = drv: {
    derivationPath = lib.safePlatformDrvEval targetSystem (drv: drv.drvPath) drv;
    outputPath = lib.safePlatformDrvEval targetSystem (drv: drv.outPath) drv;
  };

  # Helper function to find derivations in a deeply nested attribute set.
  # To be used on key-value pairs in an attribute set.
  # While recursing, it builds the attribute path to the currently evaluated key-value pair.
  # It returns either the current attribute path to the value if it is a derivation, or a deeply nested attribute set of attribute paths to derivations.
  # It should be used with `builtins.mapAttrs` and `lib.collect` to build a list of attribute paths to all derivations.
  # Args:
  #     parentPath: attribute path to the parent attribute set
  #     name: key in the currently evaluated key-value pair
  #     value: value in the currently evaluated key-value pair
  # Usage:
  #     builtins.mapAttrs (findRecursively "") (builtins.getFlake "nixpkgs")
  findRecursively =
    parentPath: key: value':
    let
      # compute attribute path to current attribute set from root attribute set
      attributePath = if key == null then null else (if parentPath == "" then "" else parentPath + ".") + key;
      value = lib.safeEval value';
    in
    if nixpkgs.lib.isDerivation value then
    # yield found derivation
    # if it has multiple output derivations, yield them instead
      let foundDrvs =
        if value ? outputs
        then (map (name: describeDrv value.${name} // { attributePath = attributePath + ".${name}"; }) value.outputs)
        else [ ((describeDrv value) // { inherit attributePath; }) ];
      in builtins.trace (builtins.toJSON { inherit foundDrvs; }) foundDrvs
    else
    # recurse when the current value is an attribute set, otherwise stop
      if (nixpkgs.lib.isAttrs value) && (value.recurseForDerivations or false)
      then
        builtins.mapAttrs (findRecursively attributePath) value
      else
        null
  ;
in
# to prevent accumlutation in memory
lib.collect (x: false) (builtins.mapAttrs (findRecursively "") targetFlakePkgs)
