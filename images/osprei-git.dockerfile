FROM rust:latest

CMD ["sh", "-c", "git clone $SOURCE /workspace/code"]
