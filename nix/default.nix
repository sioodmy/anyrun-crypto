{
  rustPlatform,
  sqlite,
}:
rustPlatform.buildRustPackage {
  pname = "cryptorun";
  version = "0.1.0";

  src = ../.;

  buildInputs = [
    sqlite
  ];

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes."anyrun-interface-0.1.0" = "sha256-SBPF3tUDGs/C8/8VXKt6Yzpq//17BnFvPZTqfHUj6NI=";
  };
}
