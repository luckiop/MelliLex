import { invoke } from '@tauri-apps/api/core';

let enabled = false;

export function setDevLogEnabled(next: boolean) {
  enabled = next;
}

export function isDevLogEnabled() {
  return enabled;
}

export function traceCommand(command: string) {
  if (!enabled) {
    return;
  }
  if (
    command === 'dev_log' ||
    command === 'log_client_error' ||
    command.includes('api_key') ||
    command.includes('settings') ||
    command.includes('provider')
  ) {
    return;
  }
  void invoke('dev_log', { message: `invoke ${command}` }).catch(() => undefined);
}

export function reportClientError(message: string) {
  void invoke('log_client_error', { message }).catch(() => undefined);
}
