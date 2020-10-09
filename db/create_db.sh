#!/usr/bin/env bash

cd "$(dirname "$0")" || exit
sqlite3 ../orchestrator.db < db.sql
