class MovieboxTui < Formula
  VERSION = "0.1.11"
  MACOS_SHA256 = "b10171528d25b2f45dd04b281535b44b38a4e1d3c483dbfe9920fddda89cca28"
  LINUX_X64_SHA256 = "b93e6441f2d96dc8746d03f33a3fa63d455bb5e2fe60daf794b37e21b6ad69ec"
  LINUX_ARM64_SHA256 = "262f5147e8f66f87a9f2206128f6f246c1df2d65ba0632f5bf2a12f0f42e5ef5"

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
