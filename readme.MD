# GitHub Analyzer

GitHub Analyzer is a collection of utility functions that retrieves data from GitHub, analyzes them, and distills findings into reports, helping users gain insight into the activities of a specific repository or a user. The result is delivered via flows-network's webhook. 

## Features

- **Weekly-Report**: This feature generates a weekly summary of activities for a specified GitHub repository. It tracks contributions, with a focus on commits, issues, and discussions within a specific timeframe. 

- **User-Specific Reporting**: This functionality allows for detailed analysis on a particular GitHub user's activities. Alternatively, it can be used to gain insights into the contributions of key community members.

- **Find About a Repo**: GitHub Analyzer retrieves data from relevant pages on GitHub and presents you a concise summary.
  
- **Find About a GitHub User**: GitHub Analyzer retrieves data from relevant pages on GitHub and presents you a concise summary.

## Usage

To use GitHub Analyzer, deploy your code on the flows-network platform, you'll get an endpoint url, make an HTTP call to the designated endpoint. For example, `https://code.flows.network/webhook/pRNFjLQGuMJ1fpEE1Us0?owner=flows-network&repo=chat-with-text&username=juntao&token=GHSAT0AAAAAACFVFOJBSGVG6ZZUN7YCRV5SZHWK22Q`.

Here is how the output might look:
```
found 2 commits:
https://github.com/flows-network/chat-with-text/commit/1c6b904b8591c836ee142d546350f5b342dfb66d
https://github.com/flows-network/chat-with-text/commit/d4a344d3cf608025b3058524ce4e9e8a0cfd08fd
found 0 issues:

The addition of the prompt feature aligns with the project’s goal of enabling the usage of saved embeddings within Telegram chats. It enhances the user experience by providing a more interactive and engaging interface for users to input their prompts in the chat.",
The addition of the prompt feature is a significant contribution as it adds a crucial functionality to…
```