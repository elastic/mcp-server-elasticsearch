# Contributing

[fork]: https://github.com/elastic/mcp-server-elasticsearch/fork
[pr]: https://github.com/elastic/mcp-server-elasticsearch/compare
[code-of-conduct]: https://www.elastic.co/community/codeofconduct

Elasticsearch MCP Server client is open source and we love to receive contributions from our community — you!

There are many ways to contribute, from writing tutorials or blog posts, improving the documentation, submitting bug reports and feature requests or writing code.


Contributions are [released](https://help.github.com/articles/github-terms-of-service/#6-contributions-under-repository-license) under the [project's license](../LICENSE).

Please note that this project follows the [Elastic's Open Source Community Code of Conduct][code-of-conduct].

## Development Setup

1. Install Node.js 18+ (using [nvm](https://github.com/nvm-sh/nvm) is recommended)
   ```bash
   nvm use
   ```
2. Install dependencies
   ```bash
   npm install
   ```
3. Build the project
   ```bash
   npm run build
   ```

## Elasticsearch for Local Development

You can use either:

1. **Elastic Cloud** - Use an existing Elasticsearch deployment and your API key
2. **Local Elasticsearch** - Run Elasticsearch locally using the [start-local](https://www.elastic.co/guide/en/elasticsearch/reference/current/run-elasticsearch-locally.html) script:
   ```bash
   curl -fsSL https://elastic.co/start-local | sh
   ```

   This starts Elasticsearch and Kibana with Docker:
   - Elasticsearch: http://localhost:9200
   - Kibana: http://localhost:5601

> [!NOTE]
> The `start-local` setup is for development only. It uses basic authentication and disables HTTPS.

## Development Workflow

1. [Fork][fork] and clone the repository
2. Create a new branch: `git checkout -b my-branch-name`
3. Make your changes
4. Test locally with the MCP Inspector:
   ```bash
   ES_URL=your-elasticsearch-url ES_API_KEY=your-api-key npm run inspector
   ```
5. Push to your fork and [submit a pull request][pr]

## Best Practices

- Follow existing code style and patterns
- Write [conventional commits](https://www.conventionalcommits.org/)
- Include tests for your changes
- Keep PRs focused on a single concern
- Update documentation as needed
- Use TypeScript with proper typing
- Add JSDoc comments to new functions

## Getting Help

- Open an issue in the repository
- Ask questions on [discuss.elastic.co](https://discuss.elastic.co/)

## Resources

- [How to Contribute to Open Source](https://opensource.guide/how-to-contribute/)
- [Using Pull Requests](https://help.github.com/articles/about-pull-requests/)
- [Elastic Code of Conduct][code-of-conduct]
