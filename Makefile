default: xcframework

.PHONY: setup objects bindings xcframework swift-build clean-swift clean-rust clean

setup:
	set -e
	mkdir -p bindings
	mkdir -p objects
	mkdir -p Sources/SwiftyTypst
	rm -rf objects/*
	rm -rf Sources/SwiftyTypst/*

objects/sim_libswiftytypst.a: setup
	cargo build --target aarch64-apple-ios-sim --release --lib  							# iOS simulator (arm64)
	cargo build --target x86_64-apple-ios --release --lib       							# iOS simulator (x86_64)
	lipo -create \
		"target/x86_64-apple-ios/release/libswiftytypst.a" \
		"target/aarch64-apple-ios-sim/release/libswiftytypst.a" \
		-output "objects/sim_libswiftytypst.a"
	codesign -f -s - "objects/sim_libswiftytypst.a"

objects/universal_libswiftytypst.a: setup
	cargo build --target aarch64-apple-darwin --release --lib   							# macOS (arm64)
	cargo build --target x86_64-apple-darwin --release --lib    							# macOS (x86_64)
	lipo -create \
		"target/aarch64-apple-darwin/release/libswiftytypst.a" \
		"target/x86_64-apple-darwin/release/libswiftytypst.a" \
		-output "objects/universal_libswiftytypst.a"
	codesign -f -s - "objects/universal_libswiftytypst.a"

objects/ios_libswiftytypst.a: setup
	cargo build --target aarch64-apple-ios --release --lib      							# iOS device (arm64)
	mv "target/aarch64-apple-ios/release/libswiftytypst.a" "objects/ios_libswiftytypst.a"
	codesign -f -s - "objects/ios_libswiftytypst.a"

objects: objects/sim_libswiftytypst.a objects/ios_libswiftytypst.a objects/universal_libswiftytypst.a

bindings:
	rm -rf bindings/*
	cargo run --bin uniffi-bindgen generate "src/typst.udl" --language swift --out-dir ./bindings
	mv bindings/SwiftyTypstFFI.modulemap bindings/module.modulemap
	mv bindings/SwiftyTypst.swift Sources/SwiftyTypst/SwiftyTypst.swift

xcframework: objects bindings
	rm -rf SwiftyTypstFFI.xcframework
	xcodebuild -create-xcframework \
		-library objects/sim_libswiftytypst.a \
		-headers bindings/ \
		-library objects/universal_libswiftytypst.a \
		-headers bindings/ \
		-library objects/ios_libswiftytypst.a \
		-headers bindings/ \
	    -output SwiftyTypstFFI.xcframework

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
