class ServerConfig:
    """
    Configuration class for the server.

    This class contains a collection of setters for configuring a Reduce server.
    The settings are write only, and can only be set once before starting the
    server.
    """
    def __init__(self):
        """
        Create a default instance for server configuration.

        .. IMPORTANT::
           Some settings are required to be set when running the server, trying
           to start the server with a default configuration will raise an error.
        """
        ...

    def database_url(self, value: str):
        """
        Set the connection URL for the database.

        .. topic:: Expected format

            postgres://{user}:{password}@{url}/{database}
        """
        ...

    def server_bind_address(self, value: str):
        """
        Set the address to bind the server to.

        You're binding to a TCP address, if you don't know know what that means you
        should either learn what networking is or ask someone else that does know.

        .. topic:: Expected format

            {ip}:{port}
        """
        ...

    def start_server(self):
        """
        Start the Reduce server.

        .. IMPORTANT::
           The lifetime of the server is tied to the instance of self.
        """
        ...
