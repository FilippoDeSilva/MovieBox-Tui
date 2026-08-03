class MovieboxTui < Formula
  VERSION = "0.1.8"
  MACOS_SHA256 = "c339c9f7dc6ee966c09fbc996a6f4b0e5e0326a24f3b1bab3c05383eba9587a4"
  LINUX_X64_SHA256 = "ed192f675cfc5249c960b35301924d0cd5b0e025dea5589e78572f11daaa7641"
  LINUX_ARM64_SHA256 = "d30fbd5cfa121bd1b77d0330df1377e3de75d9d72ae9459530770195321793ec"

  desc "Stream movies, shows, anime, and live TV from your terminal"
  homepage "https://github.com/mesamirh/MovieBox-Tui"
  version VERSION
  license "MIT"

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
