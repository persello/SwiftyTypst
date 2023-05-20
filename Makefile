default: xcframework

setup:
	set -e
	mkdir -p bindings
	mkdir -p objects
	rm -rf Typst.xcframework
	rm -rf objects/*
	rm -rf bindings/*

compile-rust: setup
	cargo build --target aarch64-apple-ios-sim --release --lib  							# iOS simulator (arm64)
	cargo build --target x86_64-apple-ios --release --lib       							# iOS simulator (x86_64)
	cargo build --target aarch64-apple-ios --release --lib      							# iOS device (arm64)
	cargo build --target aarch64-apple-darwin --release --lib   							# macOS (arm64)
	cargo build --target x86_64-apple-darwin --release --lib    							# macOS (x86_64)

sim-fat-binary: compile-rust
	lipo -create \
		"target/x86_64-apple-ios/release/libtypst_bindings.a" \
		"target/aarch64-apple-ios-sim/release/libtypst_bindings.a" \
		-output "objects/sim_libtypst_bindings.a"
	codesign -f -s - "objects/sim_libtypst_bindings.a"

macos-fat-binary: compile-rust
	lipo -create \
		"target/aarch64-apple-darwin/release/libtypst_bindings.a" \
		"target/x86_64-apple-darwin/release/libtypst_bindings.a" \
		-output "objects/universal_libtypst_bindings.a"
	codesign -f -s - "objects/universal_libtypst_bindings.a"

ios-binary: compile-rust
	mv "target/aarch64-apple-ios/release/libtypst_bindings.a" "objects/ios_libtypst_bindings.a"
	codesign -f -s - "objects/ios_libtypst_bindings.a"

objects: sim-fat-binary macos-fat-binary ios-binary

bindings: compile-rust
	cargo run --bin uniffi-bindgen generate "src/typst.udl" --language swift --out-dir ./bindings
	# Create a Swift module
	swiftc -module-name Typst \
		-emit-module -emit-module-path ./bindings \
		-parse-as-library \
		-L ./objects \
		-luniversal_libtypst_bindings \
		-Xcc -fmodule-map-file=./bindings/TypstFFI.modulemap \
		./bindings/Typst.swift

xcframework: objects bindings
	xcodebuild -create-xcframework \
		-library objects/sim_libtypst_bindings.a \
		-headers bindings/ \
		-library objects/universal_libtypst_bindings.a \
		-headers bindings/ \
		-library objects/ios_libtypst_bindings.a \
		-headers bindings/ \
		-output Typst.xcframework

clean:
	rm -rf objects
	rm -rf bindings
	rm -rf Typst.xcframework
	cargo clean