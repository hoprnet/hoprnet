# Given a source tree containing a 'flake.lock' file, it fetches the 
# repositories listed in the lock file.
# 
# This method was taken from the `flake-compat` project.
# https://github.com/edolstra/flake-compat/blob/master/default.nix#L16

{ src, }:
let
  lockFilePath = src + "/flake.lock";
  lockFile = builtins.fromJSON (builtins.readFile lockFilePath);

  fetchTree =
    info:
    if info.type == "github" then
      {
        outPath =
          fetchTarball
            ({ url = "https://api.${info.host or "github.com"}/repos/${info.owner}/${info.repo}/tarball/${info.rev}"; }
              // (if info ? narHash then { sha256 = info.narHash; } else { })
            );
        rev = info.rev;
        shortRev = builtins.substring 0 7 info.rev;
        lastModified = info.lastModified;
        narHash = info.narHash;
      }
    else if info.type == "git" then
      {
        outPath =
          builtins.fetchGit
            ({ url = info.url; }
            // (if info ? rev then { inherit (info) rev; } else { })
            // (if info ? ref then { inherit (info) ref; } else { })
            // (if info ? submodules then { inherit (info) submodules; } else { })
            );
        lastModified = info.lastModified;
        narHash = info.narHash;
      } // (if info ? rev then {
        rev = info.rev;
        shortRev = builtins.substring 0 7 info.rev;
      } else { })
    else if info.type == "path" then
      {
        outPath = builtins.path { path = info.path; };
        narHash = info.narHash;
      }
    else if info.type == "tarball" then
      {
        outPath =
          fetchTarball
            ({ inherit (info) url; }
              // (if info ? narHash then { sha256 = info.narHash; } else { })
            );
      }
    else if info.type == "gitlab" then
      {
        inherit (info) rev narHash lastModified;
        outPath =
          fetchTarball
            ({ url = "https://${info.host or "gitlab.com"}/api/v4/projects/${info.owner}%2F${info.repo}/repository/archive.tar.gz?sha=${info.rev}"; }
              // (if info ? narHash then { sha256 = info.narHash; } else { })
            );
        shortRev = builtins.substring 0 7 info.rev;
      }
    else
    # FIXME: add Mercurial, tarball inputs.
      throw "flake input has unsupported input type '${info.type}'";

in
builtins.mapAttrs
  (name: value: { outPath = fetchTree value.locked; })
  lockFile.nodes
