.PHONY: all
all: build

.PHONY: build
build:
	dotnet build informer.sln

.PHONY: test
test:
	dotnet test informer.sln
