build: 
	docker build -t spider-env .
	docker run -it --rm spider-env

clean:
	docker image rm spider-env:latest

.PHONY: build clean
