class MovieboxTui < Formula
  VERSION = "0.1.15"
  MACOS_SHA256 = "f1a72d6242fe16a9e90f403a7b266146068a59363611c5ad260a570a14554ba0"
  LINUX_X64_SHA256 = "87c48db10f269e24238f16f55115748b2153fb147a63e5bd11d7476eb206e722"
  LINUX_ARM64_SHA256 = "004e72d53993ed9d22a53e79e4cc2c7aa16395e306fc066c65ccff1d182e3d93"

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
