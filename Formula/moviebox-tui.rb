class MovieboxTui < Formula
  VERSION = "0.1.18"
  MACOS_SHA256 = "b37779a2544e27048f9d732800d1dd1aa6615b553c925991d85b98030e1baa9b"
  LINUX_X64_SHA256 = "3ff0f46ecfbafe6c90597747b722032dafc2801b08e27de80913b03474ccb73d"
  LINUX_ARM64_SHA256 = "cc0d4490ed99267b753d233bd64973d3f1b09792b297c72c9234084f19658615"

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
