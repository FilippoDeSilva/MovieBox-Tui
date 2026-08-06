class MovieboxTui < Formula
  VERSION = "0.1.10"
  MACOS_SHA256 = "d78279b3034d4c7fca49babb7834ce88497cc53f032c66f436ea20029ef6fa1f"
  LINUX_X64_SHA256 = "1192ff4a8b5d0475fe3d5a8c6016543e3f00bf432eb41d244287dd66fc6a6d47"
  LINUX_ARM64_SHA256 = "89d713037e02245cb9cc295a2702027bd413701781ecdc566c6c6779ac2f3764"

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
