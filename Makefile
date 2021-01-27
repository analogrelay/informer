ENTRYPOINT="main.go"

.PHONY: all
all: informer

bin:
	mkdir -p bin

informer: bin
	go build -o bin/informer $(ENTRYPOINT)
