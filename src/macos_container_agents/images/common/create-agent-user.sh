#!/bin/sh
set -eu

agent_uid=1000
agent_gid=1000

group_name="$(getent group "${agent_gid}" | cut -d: -f1 || true)"
if [ -n "${group_name}" ]; then
    if [ "${group_name}" != "agent" ]; then
        groupmod --new-name agent "${group_name}"
    fi
else
    groupadd --gid "${agent_gid}" agent
fi

user_name="$(getent passwd "${agent_uid}" | cut -d: -f1 || true)"
if [ -n "${user_name}" ]; then
    if [ "${user_name}" != "agent" ]; then
        usermod --login agent --home /home/agent --shell /bin/bash "${user_name}"
    else
        usermod --home /home/agent --shell /bin/bash agent
    fi
else
    useradd --uid "${agent_uid}" --gid "${agent_gid}" --create-home --shell /bin/bash agent
fi

mkdir -p /home/agent
chown -R "${agent_uid}:${agent_gid}" /home/agent
