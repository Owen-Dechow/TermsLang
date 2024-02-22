$(VERBOSE).SILENT:

trans:
	echo "Transpiling"

	echo "\n- Removing former output dir"
	rm -rf output

	echo "\n- Creating output dir"
	cargo init output

	echo "\n- Starting transpiler"
	cargo run

	echo "\n- Copying internals"
	cp -r internals/internals output/src/internals

	echo "\nDone transpiling"
build:
	echo "Building to ./a"

	echo "\n- Removing former output dir"
	rm -rf output

	echo "\n- Creating output dir"
	cargo init output

	echo "\n- Starting transpiler"
	cargo run

	echo "\n- Copying internals"
	cp -r internals/internals output/src/internals
	
	echo "\n- Building output"
	RUSTFLAGS=-Awarnings cargo build --manifest-path="./output/Cargo.toml"
	
	echo "\n- Copying output"
	cp target/debug/output ./a

	echo "\n- Removing output dir"
	rm -rf output

	echo "\nDone building (./a)"
time-comp:
	echo "Python Time:"
	time python3 test.py

	echo "Terms Time:"
	time ./a

run:
	time ./a

clean:
	echo "Cleaning"
 
	echo "\n- Update toml file"
	-sed -i '' 's/"replacement"//g' Cargo.toml

	echo "\n- Running cargo clean"
	-cargo clean

	echo "\n- Removing ./a build"
	-rm ./a	

	echo "\n- Removing output dir"
	-rm -rf output

	echo "\nDone cleaning"