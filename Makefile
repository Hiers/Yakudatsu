build: src/kanji_search.rs src/main.rs src/sentence_search.rs src/util.rs src/word_search.rs
	cargo build

release: src/kanji_search.rs src/main.rs src/sentence_search.rs src/util.rs src/word_search.rs
	cargo build --release

clean: 
	rm -r target/debug/
	rm -r target/release/

install: target/release/ykdt
	mkdir -p $(DESTDIR)/usr/bin
	mkdir -p $(DESTDIR)/usr/local/share/ykdt
	cp target/release/ykdt $(DESTDIR)/usr/bin
	cp kanji_strokes $(DESTDIR)/usr/local/share/ykdt

uninstall:
	rm $(DESTDIR)/usr/bin/ykdt
	rm -r $(DESTDIR)/usr/local/share/ykdt

.PHONY: build release install uninstall clean
