class MovieboxTui < Formula
  VERSION = "0.1.17"
  MACOS_SHA256 = "93f1f7b523f3ad880136a5ab3bd0041222eb39fac24040e0981beb75e2347549"
  LINUX_X64_SHA256 = "41ba546ee099384b87b3215dff71b0cb4306e356e68cb8b977137087a0a2acc8"
  LINUX_ARM64_SHA256 = "2b05998b89d3edb42a96e74dba6b74b8a21f85d4a265066f0ad60a1fea533d05"

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
