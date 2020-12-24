

target/release/libmylib.so: src/lib.rs
	cargo build --release

test: target/release/libmylib.so
	python3 test-python.py
	prove test-perl.pl
