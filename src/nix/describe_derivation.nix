# Describe a derivation
#
# Args (as environment variables):
#     TARGET_FLAKE_REF: flake reference to evaluate
#     TARGET_SYSTEM: system to evaluate
#     TARGET_ATTRIBUTE_PATH: attribute path to the derivation to evaluate
#     RUNTIME_ONLY: 1 if you only want to include "buildInputs" (only runtime dependencies), 0 if you want all dependencies
#
# Example:
# TARGET_FLAKE_REF="nixpkgs" TARGET_SYSTEM="x86_64-linux" TARGET_ATTRIBUTE_PATH="python3" nix eval --json --file describe-derivation.nix

let
  nixpkgs = builtins.getFlake "nixpkgs";
  lib = import <lib> { inherit nixpkgs; };

  # Arguments have to be taken from environment when using `nix` command
  targetFlakeRef = builtins.getEnv "TARGET_FLAKE_REF";
  targetAttributePath = builtins.getEnv "TARGET_ATTRIBUTE_PATH";
  targetSystem = let env = builtins.getEnv "TARGET_SYSTEM"; in if env == "" then builtins.currentSystem else env;
  # 0 is false, everything else is true
  runtimeOnly = if builtins.getEnv "RUNTIME_ONLY" == "0" then false else true;

  # Get pkgs
  targetFlake = builtins.getFlake targetFlakeRef;
  targetFlakePkgs = lib.getFlakePkgs targetFlake targetSystem;

  # Get target value
  targetValue = lib.getValueAtPath targetFlakePkgs targetAttributePath;
in
{
  name = targetValue.name;
  parsed_name = (builtins.parseDrvName targetValue.name);
  attribute_path = targetAttributePath;

  src =
    if targetValue ? src.gitRepoUrl && targetValue ? src.rev
    then
      {
        git_repo_url = targetValue.src.gitRepoUrl;
        rev = targetValue.src.rev;
      }
    else
      null;

  nixpkgs_metadata =
    {
      description = (builtins.tryEval (targetValue.meta.description or "")).value;
      pname = (builtins.tryEval (targetValue.pname or "")).value or null;
      version = (builtins.tryEval (targetValue.version or "")).value;
      broken = (builtins.tryEval (targetValue.meta.broken or false)).value;
      homepage = (builtins.tryEval (targetValue.meta.homepage or "")).value;
      licenses = (builtins.tryEval (
        if builtins.isAttrs (targetValue.meta.license or null)
        # In case the license attribute is not a list, we produce a singleton list to be consistent
        then [{
          spdx_id = targetValue.meta.license.spdxId or null;
          full_name = targetValue.meta.license.fullName or null;
        }]
        # In case the license attribute is a list
        else if builtins.isList (targetValue.meta.license or null)
        then
          builtins.map
            (l: {
              spdx_id = l.spdxId or null;
              full_name = l.fullName or null;
            })
            targetValue.meta.license
        else null
      )).value;
    };

  # path to the evaluated derivation file
  derivation_Path = lib.safePlatformDrvEval targetSystem (drv: drv.drvPath) targetValue;

  # path to the realized (=built) derivation
  # note: we can't name it `outPath` because serialization would only output it instead of dict, see Nix `toString` docs
  output_path =
    # TODO meaningfully represent when it's not the right platform (instead of null)
    lib.safePlatformDrvEval
      targetSystem
      (drv: drv.outPath)
      targetValue;
  outputs = map (name: { inherit name; output_path = lib.safePlatformDrvEval targetSystem (drv: drv.outPath) targetValue.${name}; }) (targetValue.outputs or [ ]);
  build_inputs = nixpkgs.lib.lists.flatten
    (map
      (inputType:
        map
          (elem:
            {
              build_input_type = nixpkgs.lib.removeSuffix "s" (lib.toSnakeCase inputType);
              attribute_path = targetAttributePath + ".${inputType}.${builtins.toString elem.index}";
              output_path = lib.safePlatformDrvEval targetSystem (drv: drv.outPath) elem.value;
            }
          )
          (
            # only keep derivations in inputs
            # TODO include path objects
            builtins.filter
              (elem: nixpkgs.lib.isDerivation elem.value)
              (lib.enumerate (targetValue.${inputType} or [ ]))
          )
      )
      (
        [ "buildInputs" "propagatedBuildInputs" ]
        ++ nixpkgs.lib.optionals (!runtimeOnly) [ "nativeBuildInputs" ]
      )
    );
}
