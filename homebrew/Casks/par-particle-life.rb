cask "par-particle-life" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.1.0"
  sha256 arm:   "PLACEHOLDER",
         intel: "PLACEHOLDER"

  url "https://github.com/paulrobello/par-particle-life/releases/download/v#{version}/par-particle-life-macos-#{arch}.zip"
  name "par-particle-life"
  desc "GPU-accelerated particle life simulation in Rust using wgpu"
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
