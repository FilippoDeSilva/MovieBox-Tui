class MovieboxTui < Formula
  VERSION = "0.1.16"
  MACOS_SHA256 = "b0947a93b6fab73e5858858b9bd560ce85e4c02bff485e418c0121f2c18c3949"
  LINUX_X64_SHA256 = "e8663b222f1c5daf2e04134af89ef9ffac2954c26e5afa706b8acc494588d0ca"
  LINUX_ARM64_SHA256 = "7758097653673caad90b7b32e508517264b8d415af8c41a05639898f73c5adc9"

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
