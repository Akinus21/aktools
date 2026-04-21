class Aktools < Formula
  desc "Modular CLI tool runner with module registry"
  homepage "https://github.com/Akinus21/aktools"
  url "https://github.com/Akinus21/aktools/releases/download/v1.00/aktools-v1.00-linux-x86_64.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"

  depends_on "curl"

  def install
    bin.install "aktools"
  end

  test do
    system "#{bin}/aktools", "--version"
  end
end