{
  description = "Rust SDK foundation for the Qualisys QTM real-time protocol";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      lib = nixpkgs.lib;
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = lib.genAttrs supportedSystems;
      pkgsFor = system: import nixpkgs { inherit system; };
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "qualisys-rust-sdk";
            version = "0.1.0";

            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            __darwinAllowLocalNetworking = true;

            cargoBuildFlags = [ "--all-targets" ];
            cargoTestFlags = [ "--all-targets" ];

            meta = {
              description = "Rust SDK foundation for the Qualisys QTM real-time protocol";
              license = with pkgs.lib.licenses; [
                mit
                asl20
              ];
            };
          };
        }
      );

      checks = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          package = self.packages.${system}.default;
          source = pkgs.lib.cleanSource ./.;
        in
        {
          inherit package;

          rustfmt = pkgs.runCommand "qualisys-rustfmt" { nativeBuildInputs = [ pkgs.cargo pkgs.rustfmt ]; } ''
            cp -R ${source} source
            chmod -R u+w source
            cd source
            cargo fmt --all -- --check
            touch "$out"
          '';

          simulator-e2e = pkgs.runCommand "qualisys-simulator-e2e" {
            nativeBuildInputs = [ package pkgs.python3 ];
            __darwinAllowLocalNetworking = true;
          } ''
            port="$(
              python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
            )"

            qualisys-sim --bind "127.0.0.1:$port" --hz 240 --rigid-bodies 2 >simulator.log 2>&1 &
            sim_pid="$!"
            trap 'kill "$sim_pid" 2>/dev/null || true; wait "$sim_pid" 2>/dev/null || true' EXIT

            for attempt in $(seq 1 50); do
              if qualisys-rt --port "$port" --timeout-ms 1000 info >info.out 2>info.err; then
                break
              fi
              if [ "$attempt" -eq 50 ]; then
                cat simulator.log >&2 || true
                cat info.err >&2 || true
                exit 1
              fi
              sleep 0.1
            done

            grep -q "qtm_version: QTM RT simulator 0.1" info.out
            grep -q "byte_order: little endian" info.out

            qualisys-rt --port "$port" --timeout-ms 1000 params --parameters general,6d >params.xml
            grep -q "<Frequency>240</Frequency>" params.xml
            grep -q "sim_body_1" params.xml
            grep -q "sim_body_2" params.xml

            qualisys-rt --port "$port" --timeout-ms 1000 frame --components 6dres >frame.out
            grep -q "frame=1 " frame.out
            grep -q "SixDResidual: 2 bodies" frame.out

            qualisys-rt --port "$port" --timeout-ms 1000 stream --components 6dres --transport udp --count 3 >stream-udp.out
            test "$(grep -c '^frame=' stream-udp.out)" -eq 3
            grep -q "SixDResidual=2 bodies" stream-udp.out

            qualisys-rt --port "$port" --timeout-ms 1000 stream --components 6dres --transport tcp --count 3 >stream-tcp.out
            test "$(grep -c '^frame=' stream-tcp.out)" -eq 3
            grep -q "SixDResidual=2 bodies" stream-tcp.out

            mkdir -p "$out"
            cp simulator.log info.out params.xml frame.out stream-udp.out stream-tcp.out "$out"/
          '';

          example-e2e = pkgs.rustPlatform.buildRustPackage {
            pname = "qualisys-example-e2e";
            version = "0.1.0";

            src = source;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkgs.python3 ];
            __darwinAllowLocalNetworking = true;

            cargoBuildFlags = [
              "--bin"
              "qualisys-sim"
              "--example"
              "rt_udp_6d"
            ];
            doCheck = false;

            installPhase = ''
              runHook preInstall

              sim_bin="$(find target -type f -perm -0100 -name qualisys-sim | head -n 1)"
              example_bin="$(find target -type f -perm -0100 -path '*/examples/rt_udp_6d' | head -n 1)"
              test -x "$sim_bin"
              test -x "$example_bin"

              port="$(
                python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
              )"

              "$sim_bin" --bind "127.0.0.1:$port" --hz 240 --rigid-bodies 2 >simulator.log 2>&1 &
              sim_pid="$!"
              trap 'kill "$sim_pid" 2>/dev/null || true; wait "$sim_pid" 2>/dev/null || true' EXIT

              for attempt in $(seq 1 50); do
                if "$example_bin" 127.0.0.1 "$port" 3 >example.out 2>example.err; then
                  break
                fi
                if [ "$attempt" -eq 50 ]; then
                  cat simulator.log >&2 || true
                  cat example.err >&2 || true
                  exit 1
                fi
                sleep 0.1
              done

              grep -q "Connected to QTM version: QTM RT simulator 0.1" example.out
              grep -q "Listening for RT packets on UDP" example.out
              test "$(grep -c '^frame=' example.out)" -eq 3
              grep -q "body_count=2" example.out

              mkdir -p "$out"
              cp simulator.log example.out "$out"/

              runHook postInstall
            '';
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.rustc
              pkgs.rustfmt
            ];
          };
        }
      );
    };
}
