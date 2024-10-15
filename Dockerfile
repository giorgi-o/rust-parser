# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the current directory contents into the container
COPY . .

# Build the Rust application
RUN cargo build

# Run the specified Rust file
CMD ["cargo", "run", "--", "example_input_source_code/full.txt"]
