import { createApp, defineComponent, h } from 'vue'

export function withSetup<T>(composable: () => T): T {
  let result: T

  const App = defineComponent({
    setup() {
      result = composable()
      return () => null
    },
  })

  const app = createApp(App)
  const root = document.createElement('div')
  app.mount(root)
  app.unmount()

  return result!
}
