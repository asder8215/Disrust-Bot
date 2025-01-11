# # Start with the base Shuttle image
# FROM shuttle/worker:latest

# # Install nasm and other dependencies
# RUN apt-get update && apt-get install -y nasm

# # Set up the working directory
# WORKDIR /app

# # Copy the necessary files
# COPY . .

# # Build the project
# RUN cargo build --release

# # Specify the command to run your bot
# CMD ["cargo", "run"]