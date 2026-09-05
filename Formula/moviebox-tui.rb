class MovieboxTui < Formula
  VERSION = "0.1.16"
  MACOS_SHA256 = "6eb72888eab6142861eb6667d13c029da110a39a9d8c3e36fa16a409f5eb6a76"
  LINUX_X64_SHA256 = "a6e045aff596acb6995e938586bf93beb8ed7289e3dc969252ee3a8c68396739"
  LINUX_ARM64_SHA256 = "8fa1e0409ef9b785f3aca351d4ef1f8ef25a0ff92761e9c65f7b3872cae57ea7"

  desc "Stream movies, shows, anime, and live TV from your terminal"
  homepage "https://github.com/mesamirh/MovieBox-Tui"
  version VERSION
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    url "https://github.com/mesamirh/MovieBox-Tui/releases/download/v#{VERSION}/MovieBox_macOS_Universal.tar.gz"
    sha256 MACOS_SHA256
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/mesamirh/MovieBox-Tui/releases/download/v#{VERSION}/MovieBox_Linux_arm64.tar.gz"
      sha256 LINUX_ARM64_SHA256
    else
      url "https://github.com/mesamirh/MovieBox-Tui/releases/download/v#{VERSION}/MovieBox_Linux_x64.tar.gz"
      sha256 LINUX_X64_SHA256
    end
  end

  def install
    bin.install "moviebox-tui"
  end

  test do
    system "#{bin}/moviebox-tui", "--version"
  end
end
