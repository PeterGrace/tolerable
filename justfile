commit := `git rev-parse HEAD`
transport := "docker://"
registry := "foobar"
image := "library/tolerable"


build:
  cross build --release --target aarch64-unknown-linux-gnu
  cross build --release --target x86_64-unknown-linux-gnu
make-image:
  docker buildx build -f Dockerfile.amd64 --push --platform linux/amd64,linux/arm64/v8 -t {{registry}}/{{image}}:{{commit}} -t {{registry}}/{{image}}:latest .

