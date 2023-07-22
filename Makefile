default: swift-build

.PHONY: setup sim-fat-binary macos-fat-binary catalyst-fat-binary ios-binary objects bindings xcframework swift-build clean-swift clean-rust clean

setup:
	set -e
	mkdir -p bindings
	mkdir -p objects
	mkdir -p Sources/SwiftyTypst
	rm -rf SwiftyTypstFFI.xcframework
	rm -rf objects/*
	rm -rf bindings/*
	rm -rf Sources/SwiftyTypst/*

sim-fat-binary: setup
	cargo build --target aarch64-apple-ios-sim --release --lib  							# iOS simulator (arm64)
	cargo build --target x86_64-apple-ios --release --lib       							# iOS simulator (x86_64)
	lipo -create \
		"target/x86_64-apple-ios/release/libtypst_bindings.a" \
		"target/aarch64-apple-ios-sim/release/libtypst_bindings.a" \
		-output "objects/sim_libtypst_bindings.a"
	codesign -f -s - "objects/sim_libtypst_bindings.a"

macos-fat-binary: setup
	cargo build --target aarch64-apple-darwin --release --lib   							# macOS (arm64)
	cargo build --target x86_64-apple-darwin --release --lib    							# macOS (x86_64)
	lipo -create \
		"target/aarch64-apple-darwin/release/libtypst_bindings.a" \
		"target/x86_64-apple-darwin/release/libtypst_bindings.a" \
		-output "objects/universal_libtypst_bindings.a"
	codesign -f -s - "objects/universal_libtypst_bindings.a"

catalyst-fat-binary: setup
	cargo build --target aarch64-apple-ios-macabi --release --lib -Zbuild-std				# Mac Catalyst (arm64)
	cargo build --target x86_64-apple-ios-macabi --release --lib -Zbuild-std				# Mac Catalyst (x86_64)
	lipo -create \
		"target/aarch64-apple-ios-macabi/release/libtypst_bindings.a" \
		"target/x86_64-apple-ios-macabi/release/libtypst_bindings.a" \
		-output "objects/catalyst_libtypst_bindings.a"
	codesign -f -s - "objects/catalyst_libtypst_bindings.a"

ios-binary: setup
	cargo build --target aarch64-apple-ios --release --lib      							# iOS device (arm64)
	mv "target/aarch64-apple-ios/release/libtypst_bindings.a" "objects/ios_libtypst_bindings.a"
	codesign -f -s - "objects/ios_libtypst_bindings.a"

objects: sim-fat-binary ios-binary catalyst-fat-binary macos-fat-binary

bindings:
	cargo run --bin uniffi-bindgen generate "src/typst.udl" --language swift --out-dir ./bindings
	mv bindings/SwiftyTypstFFI.modulemap bindings/module.modulemap
	mv bindings/SwiftyTypst.swift Sources/SwiftyTypst/SwiftyTypst.swift

xcframework: objects bindings
	xcodebuild -create-xcframework \
		-library objects/sim_libtypst_bindings.a \
		-headers bindings/ \
		-library objects/catalyst_libtypst_bindings.a \
		-headers bindings/ \
		-library objects/ios_libtypst_bindings.a \
		-headers bindings/ \
		-library objects/universal_libtypst_bindings.a \
		-headers bindings/ \
		-output SwiftyTypstFFI.xcframework

swift-build: xcframework
	swift build

clean-swift:
	rm -rf .build
	rm -rf .swiftpm
	rm -rf bindings
	rm -rf objects
	rm -rf SwiftyTypstFFI.xcframework
	rm -rf Sources

clean-rust:
	cargo clean

clean: clean-swift clean-rust