{
  description = "schizoid - multiplayer game";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust
            cargo
            rustfmt
            clippy
            pkg-config
            wayland
            libxkbcommon
            vulkan-loader
            systemd
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXinerama
            xorg.libXi
            xorg.libXxf86vm
            xorg.libXext
            xorg.libXrender
            libxcb
            alsa-lib
          ];
          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.wayland}/lib/pkgconfig:${pkgs.libxkbcommon}/lib/pkgconfig:${pkgs.systemd}/lib/pkgconfig:${pkgs.alsa-lib}/lib/pkgconfig:$PKG_CONFIG_PATH"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [pkgs.wayland pkgs.libxkbcommon pkgs.vulkan-loader pkgs.systemd pkgs.alsa-lib]}:$LD_LIBRARY_PATH"
          '';
        };
      }
    );
}
