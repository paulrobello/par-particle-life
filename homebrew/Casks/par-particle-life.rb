cask "par-particle-life" do
arch arm: "aarch64", intel: "x86_64"

version "0.1.0"
sha256 arm:   "15a49fcd1a7d6a7cbcd3acff7dc199945887f53d2780deeadaf4a35d294ad2d7",
       intel: "55d96ed845a584396b63bc0ba2b65403525516f671513a2b2708603ce7392bab"

url "https://github.com/paulrobello/par-particle-life/releases/download/v#{version}/par-particle-life-macos-#{arch}.zip"
name "par-particle-life"
desc "GPU-accelerated particle life simulation in Rust"
homepage "https://github.com/paulrobello/par-particle-life"

depends_on macos: ">= :catalina"

livecheck do
  url :homepage
  strategy :github_latest
end

app "par-particle-life.app"

zap trash: [
  "~/Library/Application Support/par-particle-life",
  "~/Library/Preferences/com.paulrobello.par-particle-life.plist",
  "~/Library/Saved Application State/com.paulrobello.par-particle-life.savedState",
]
end
