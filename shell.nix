{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    name = "testing";
    # nativeBuildInputs is usually what you want -- tools you need to run
    nativeBuildInputs = with pkgs.buildPackages; [
      cowsay
      lolcat
      kubectl
      google-cloud-sdk # gcloud cli
      jq
      yq
    ];

    SOME_ENV = "test";
    LANGUAGE = "en_US.UTF-8";
	  LC_ALL = "en_US.UTF-8";

    shellHook = ''
    cowsay "yo dawg, welcome to the project! We have installed some tools for ya!" | lolcat
    echo "tool versions:"
    kubectl version
    gcloud --version
    jq --version
    yq --version
    '';
}
