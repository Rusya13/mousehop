# Nix Flake Usage

## Run

```bash
nix run github:rusya13/mousehop

# With params
nix run github:rusya13/mousehop -- --help

```

## Home-manager module

Add input:

```nix
inputs = {
    mousehop.url = "github:rusya13/mousehop";
}
```

Optional: once a [cachix cache](https://app.cachix.org/cache/mousehop) is set up
for this fork, add it as a binary cache for faster package installs.

```nix
nixConfig = {
    extra-substituters = [
        "https://mousehop.cachix.org/"
    ];
    extra-trusted-public-keys = [
        # TODO: add the public key for the `mousehop` cachix cache once it
        # exists, e.g. "mousehop.cachix.org-1:<public-key>"
    ];
};
```

Enable mousehop:

``` nix
{
  inputs,
  ...
}: {
  # Add the Home Manager module
  imports = [inputs.mousehop.homeManagerModules.default];

  programs.mousehop = {
    enable = true;
    # systemd = false;
    # package = inputs.mousehop.packages.${pkgs.stdenv.hostPlatform.system}.default
    # Optional configuration in nix syntax, see config.toml for available options
    # settings = { };
    };
  };
}

```
