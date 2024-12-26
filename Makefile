build: src/kanji_search.rs src/main.rs src/sentence_search.rs src/util.rs src/word_search.rs
	cargo build

release: src/kanji_search.rs src/main.rs src/sentence_search.rs src/util.rs src/word_search.rs
	cargo build --release

clean:
ifeq ($(OS),Windows_NT)
	powershell "if (Test-Path .\target\debug) { rm -r .\target\debug };\
	if (Test-Path .\target\release) { rm -r .\target\release }"
else
	rm -rf target/debug/
	rm -rf target/release/
endif

install: target/release/ykdt*
ifeq ($(OS),Windows_NT)
	powershell "if (-Not (Test-Path $(DESTDIR)\"C:\Program Files\ykdt\")) { mkdir $(DESTDIR)\"C:\Program Files\ykdt\" };\
	[Environment]::SetEnvironmentVariable(\"Path\", $$env:Path + \";C:\Program Files\ykdt\", \"Machine\");\
	cp target\release\ykdt.exe $(DESTDIR)\"C:\Program Files\ykdt\";\
	cp kanji_strokes $(DESTDIR)\"C:\Program Files\ykdt\""
else
	mkdir -p $(DESTDIR)/usr/bin
	mkdir -p $(DESTDIR)/usr/local/share/ykdt
	cp target/release/ykdt $(DESTDIR)/usr/bin
	cp kanji_strokes $(DESTDIR)/usr/local/share/ykdt
endif

uninstall:
ifeq ($(OS),Windows_NT)
	powershell $$path = "([System.Environment]::GetEnvironmentVariable(\"PATH\", \"Machine\").Split(\";\") | Where-Object { $$_ -ne \"C:\Program Files\ykdt\" }) -join \";\";\
		[System.Environment]::SetEnvironmentVariable(\"PATH\",$$path,\"Machine\");\
	if (Test-Path $(DESTDIR)\"C:\Program Files\ykdt\") { rm -r $(DESTDIR)\"C:\Program Files\ykdt\" }"
else
	rm $(DESTDIR)/usr/bin/ykdt
	rm -r $(DESTDIR)/usr/local/share/ykdt
endif

.PHONY: build release install uninstall clean
