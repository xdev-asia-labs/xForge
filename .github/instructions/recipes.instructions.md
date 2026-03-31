---
description: "Use when creating or modifying recipe YAML files or Ansible playbooks for xForge infrastructure recipes."
applyTo: "recipes/**"
---
# Recipe Conventions

## recipe.yaml Structure
```yaml
name: recipe-name          # kebab-case, matches directory name
version: "1.0"             # quoted string
description: "What it does"
params:                    # optional
  - name: param_name       # snake_case
    type: string           # string, boolean, number
    default: "value"       # optional default
requires:                  # optional
  min_servers: 1
  os: "ubuntu-22.04+"
playbook: playbook.yml
tags:                      # optional, for UI filtering
  - category
```

## Ansible Playbook Conventions
- Target `hosts: all` — inventory is generated dynamically by xForge
- Use `become: true` for system operations
- Define recipe params as vars with Jinja2 defaults: `"{{ param | default('value') }}"`
- Include handlers for service restarts
- Use `apt` module for Ubuntu package management
