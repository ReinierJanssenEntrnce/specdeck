#!/bin/sh
# Prints the Homebrew cask for a release (design decision 13).
# Usage: scripts/render-cask.sh 1.0.0 <sha256 of SpecDeck-arm64.dmg> <sha256 of SpecDeck-x64.dmg> [owner/repo]
set -eu
version=$1
sha_arm=$2
sha_intel=$3
repo=${4:-nickhartjes/specdeck}

cat <<CASK
cask "specdeck" do
  arch arm: "arm64", intel: "x64"

  version "$version"
  sha256 arm:   "$sha_arm",
         intel: "$sha_intel"

  url "https://github.com/$repo/releases/download/v#{version}/SpecDeck-#{arch}.dmg"
  name "SpecDeck"
  desc "Run a coding agent with your team watching along"
  homepage "https://github.com/$repo"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :catalina"

  app "SpecDeck.app"

  zap trash: [
    "~/Library/Application Support/com.entrnce.specdeck",
    "~/Library/Caches/com.entrnce.specdeck",
    "~/Library/Saved Application State/com.entrnce.specdeck.savedState",
    "~/Library/WebKit/com.entrnce.specdeck",
  ]
end
CASK
