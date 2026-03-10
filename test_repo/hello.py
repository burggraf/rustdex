def say_hello(name):
    """Greets the user."""
    return f"Hello, {name}!"

class Greeter:
    def __init__(self, name):
        self.name = name
    
    def greet(self):
        return say_hello(self.name)
