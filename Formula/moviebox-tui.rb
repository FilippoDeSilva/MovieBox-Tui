class MovieboxTui < Formula
  VERSION = "0.1.12"
  MACOS_SHA256 = "43b226c381c1644e5d62ed3e40e8da0a8fb270f297ddefe31b337b9992851e6a"
  LINUX_X64_SHA256 = "0ef996ed850430d22efbf12d4714390a907af688a1d94fefff6e5796145ab2af"
  LINUX_ARM64_SHA256 = "c1a0a644d887bab03c78fb6f8397cced2bc32ad89190f9e00f8556887f17ca3f"

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
