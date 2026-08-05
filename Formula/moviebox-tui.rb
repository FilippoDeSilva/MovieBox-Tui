class MovieboxTui < Formula
  VERSION = "0.1.9"
  MACOS_SHA256 = "4cae8fdb2586b86e9af2b843f08a2cdd3c337f9e32585fdcc90ffaceb60af8f8"
  LINUX_X64_SHA256 = "df2786a6c219e540a192e2c4503fb324252dd576d425a7699288d0e78e69df68"
  LINUX_ARM64_SHA256 = "a5237cd76cf836c2c986233b5e5ab972329736d9e1af650385caf7db0cb0a1e5"

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
