import { defineConfig } from '@hey-api/openapi-ts'
import { getConfig } from './config/unified.config'

// Get unified configuration
const config = getConfig()

export default defineConfig({
  input: config.openapi.input,
  output: config.openapi.output,
  // Optional: Add parser configuration if needed
  // parser: {
  //   // Custom parser options
  // },
  // Optional: Add plugins configuration if needed
  // plugins: [
  //   // Custom plugins
  // ],
}) 