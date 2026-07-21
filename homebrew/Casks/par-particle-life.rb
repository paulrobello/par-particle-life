cask "par-particle-life" do
arch arm: "aarch64", intel: "x86_64"

# NOTE: The version and sha256 below are placeholders. The
# publish-homebrew-cask-core.yml workflow rewrites this file in full on every
# release, computing the real sha256 from the just-uploaded macOS zip assets
# and replacing both `version` and the `sha256 arm:/intel:` pairs. Do not edit
# them by hand unless you are intentionally cutting a manual release.
version "0.3.0"
sha256 arm:   "0000000000000000000000000000000000000000000000000000000000000000",
       intel: "0000000000000000000000000000000000000000000000000000000000000000"

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
