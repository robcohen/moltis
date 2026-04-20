# Session Context

## User Prompts

### Prompt 1

Looking at hermes in ~/code/hermes-agent I see it comes with tons of skills already, the loader shows:

██╗  ██╗███████╗██████╗ ███╗   ███╗███████╗███████╗       █████╗  ██████╗ ███████╗███╗   ██╗████████╗
██║  ██║██╔════╝██╔══██╗████╗ ████║██╔════╝██╔════╝      ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝
███████║█████╗  ██████╔╝██╔████╔██║█████╗  ███████╗█████╗███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║
██╔══██║██╔══╝  ██╔══██╗██║╚██╔╝██║██╔══╝  ╚════██║╚════╝██╔══██║██║   ██║██╔══╝  ██║╚██╗█...

### Prompt 2

I would rather have something like assets for the web-ui (html, etc), with an external directory where we just copy hermes skills (and other skills I'll find online and I "vouched" for security issues), each skill could have metadata for its original source/origin. And have all those skills enabled by default since they've been secured.

### Prompt 3

please proceed

### Prompt 4

commit push create a PR

### Prompt 5

You can now add all hermes skills by default, just copy them over from the local hermes clone I gave you

### Prompt 6

This skill is specific to hermes: crates/skills/src/assets/devops/webhook-subscriptions/SKILL.md but since we have webhooks in moltis you should be able to update it for moltis

### Prompt 7

Look at every skills and see which ones are actually depending on hermes or another tool, I think Moltis skills include frontmatter data to list required tools (cli, etc) for the skill to work, we could include that.

### Prompt 8

Please cleanup hermes sections, and use the moltis skill metadata fields to add binary expectations if any

### Prompt 9

Merge main to this branch, commit and push

### Prompt 10

I see +160k lines, where are they from?

### Prompt 11

How can we know those are safe?

### Prompt 12

but those are from the hermes repo, so I guess it's fine to fully copy them over

### Prompt 13

anything else to improve? Can you look at ~/code/openclaw and see if there are bundled skills?

### Prompt 14

yes please, also having categories in skills make sense, can you do that and add the web-ui component to view categories?

### Prompt 15

Should you add tests for some of those default skills?

### Prompt 16

Look at openclaw and hermes code, how are they adding skills to system prompts? Do they use a search skill like I have (not default enabled I think)? Because that's a lot of skills.

### Prompt 17

Maybe a format fallback would be nice?

