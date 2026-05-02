// 用途：全局命令注册表，为命令面板提供可搜索的命令列表
import { shallowRef } from "vue";

export interface Command {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  section?: string;
  action: () => void | Promise<void>;
}

const commands = shallowRef<Command[]>([]);

export function useCommands() {
  function register(cmd: Command) {
    const next = commands.value.filter((c) => c.id !== cmd.id);
    commands.value = [...next, cmd];
  }

  function unregister(id: string) {
    commands.value = commands.value.filter((c) => c.id !== id);
  }

  function getAll() {
    return commands.value;
  }

  function search(query: string): Command[] {
    if (!query) return commands.value;
    const q = query.toLowerCase();
    return commands.value.filter(
      (c) =>
        c.label.toLowerCase().includes(q) || c.id.toLowerCase().includes(q),
    );
  }

  return { commands, register, unregister, getAll, search };
}
