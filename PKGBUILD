pkgname=hustle
pkgver=1.2.1
pkgrel=1
makedepends=('rust' 'cargo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
pkgdesc="A terminal-based wordle clone and wordle solver written in rust, geared towards speedrunning"
url="https://github.com/lennonokun/hustle"
license=('MIT')

prepare() {
	cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
	export RUSTUP_TOOLCHAIN=stable
	export CARGO_TARGET_DIR=target
	cargo build --frozen --release --all-features
}

check() {
	return 0
}

package() {
	cd "$pkgname"
	# binary
	install -Dm0755 -t "$pkgdir/usr/bin" "target/release/$pkgname"
	# data
	install -Dm0644 -t "$pkgdir/usr/share/hustle/bank1.csv" "data/bank1.csv"
	install -Dm0644 -t "$pkgdir/usr/share/hustle/bank2.csv" "data/bank2.csv"
	install -Dm0644 -t "$pkgdir/usr/share/hustle/happrox.csv" "data/happrox.csv"
	# misc
	install -Dm0644 -t "$pkgdir/usr/share/licenses/$pkgname/LICENSE" "LICENSE"
	install -Dm0644 -t "$pkgdir/usr/share/doc/$pkgname" "README.md"
}
