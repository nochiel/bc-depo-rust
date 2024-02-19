# BlockchainCommons Server

BlockchainCommons Server (BC-Server) is the reusable lightweight server codebase for all BlockchainCommons projects.
The goal of this project is to allow any BlockchainCommons project to expose its API as a HTTP service in a standardized and easy way.
BlockchainCommons has several projects as commandline tools which could be made more accessible for testing and useful if a server is running that exposes their functionality.
BC-Server will expose its functionality using a JSON-RPC interface
To this end, we have the following requirements:

- A BlockchainCommons command line tool should be able to describe a simple manifest specifying
  - named endpoints (path and parameters)
  - the command to be executed when an endpoint is called.
- The manifest should be written as a rust library and can contain arbitrary logic to process input and generate output.
  - This library can then be put into a "modules" directory and the server will automatically make its functionality available.

## APIs

The following APIs are implemented:

- bc-depository: secure storage and retrieval of binary objects.

## References

- [Core Lightning](https://github.com/ElementsProject/lightning) (aka `lightningd`) has a module/plugin system that will server as a model for BC-Server.
  - See [A day in the life of a plugin
    ](https://github.com/ElementsProject/lightning/blob/master/doc/developers-guide/plugin-development/a-day-in-the-life-of-a-plugin.md)
