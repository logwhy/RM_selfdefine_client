import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { readFromStorage, writeToStorage } from '../utils/storage'

export type HudElementType = 'line' | 'rect' | 'circle' | 'text' | 'bar' | 'statusLight'
export type HudTool = 'select' | HudElementType
export type HudDataBinding =
  | 'none'
  | 'GameStatus.stage_countdown_sec'
  | 'RobotDynamicStatus.hp'
  | 'RobotDynamicStatus.heat'
  | 'RobotDynamicStatus.chassis_energy'
  | 'RobotDynamicStatus.buffer_energy'
  | 'RobotStaticStatus.robot_id'
  | 'mqttConnected'
  | 'fps'

export interface HudElement {
  id: string
  type: HudElementType
  x: number
  y: number
  w: number
  h: number
  text?: string
  color: string
  warnColor?: string
  dangerColor?: string
  autoColor?: boolean
  warnThreshold?: number
  dangerThreshold?: number
  binding: HudDataBinding
  value?: number
}

export interface HudTemplate {
  name: string
  elements: HudElement[]
}

const STORAGE_KEY = 'hero-deploy-hud-template'
const HISTORY_LIMIT = 60

function cloneElements(elements: HudElement[]) {
  return elements.map((element) => ({ ...element }))
}

function createElement(type: HudElementType, x = 44, y = 44, color = '#00e5ff'): HudElement {
  const id = `${type}-${Date.now()}-${Math.random().toString(16).slice(2)}`
  const base = { id, type, x, y, w: 180, h: 72, color, binding: 'none' as HudDataBinding }
  if (type === 'line') return { ...base, w: 220, h: 0 }
  if (type === 'circle') return { ...base, w: 86, h: 86 }
  if (type === 'text') return { ...base, w: 220, h: 34, text: 'HUD TEXT' }
  if (type === 'bar') {
    return {
      ...base,
      w: 220,
      h: 20,
      binding: 'RobotDynamicStatus.hp',
      value: 70,
      autoColor: true,
      warnColor: '#ffc93a',
      dangerColor: '#ff3045',
      warnThreshold: 0.5,
      dangerThreshold: 0.25,
    }
  }
  if (type === 'statusLight') return { ...base, w: 26, h: 26, binding: 'mqttConnected' }
  return base
}

const defaultHeroTemplate: HudTemplate = {
  name: '默认英雄',
  elements: [
    { id: 'hero-countdown', type: 'text', x: 330, y: 82, w: 210, h: 34, text: 'COUNTDOWN', color: '#ffc93a', binding: 'GameStatus.stage_countdown_sec' },
    { id: 'hero-hp', type: 'bar', x: 330, y: 128, w: 220, h: 22, color: '#2cff8c', warnColor: '#ffc93a', dangerColor: '#ff3045', autoColor: true, warnThreshold: 0.5, dangerThreshold: 0.25, binding: 'RobotDynamicStatus.hp', value: 85 },
    { id: 'hero-link', type: 'statusLight', x: 330, y: 166, w: 24, h: 24, color: '#00e5ff', binding: 'mqttConnected' },
  ],
}

const minimalTemplate: HudTemplate = {
  name: '极简比赛',
  elements: [
    { id: 'minimal-fps', type: 'text', x: 330, y: 82, w: 120, h: 30, text: 'FPS', color: '#00e5ff', binding: 'fps' },
    { id: 'minimal-hp', type: 'bar', x: 330, y: 120, w: 170, h: 16, color: '#2cff8c', warnColor: '#ffc93a', dangerColor: '#ff3045', autoColor: true, warnThreshold: 0.5, dangerThreshold: 0.25, binding: 'RobotDynamicStatus.hp', value: 85 },
  ],
}

const migratedPositions: Record<string, Pick<HudElement, 'x' | 'y'>> = {
  'hero-countdown': { x: 330, y: 82 },
  'hero-hp': { x: 330, y: 128 },
  'hero-link': { x: 330, y: 166 },
  'minimal-fps': { x: 330, y: 82 },
  'minimal-hp': { x: 330, y: 120 },
}

function separateLeftRail(elements: HudElement[]) {
  let changed = false
  const migrated = elements.map((element) => {
    const nextPosition = migratedPositions[element.id]
    if (!nextPosition || element.x > 300) return element
    changed = true
    return { ...element, ...nextPosition }
  })
  return { elements: migrated, changed }
}

export const useHudEditorStore = defineStore('hudEditor', () => {
  const editMode = ref(false)
  const locked = ref(false)
  const activeTool = ref<HudTool>('select')
  const currentColor = ref('#00e5ff')
  const selectedId = ref<string | null>(null)
  const templateName = ref(defaultHeroTemplate.name)
  const elements = ref<HudElement[]>(cloneElements(defaultHeroTemplate.elements))
  const undoStack = ref<HudElement[][]>([])
  const redoStack = ref<HudElement[][]>([])

  const selectedElement = computed(() => elements.value.find((item) => item.id === selectedId.value) ?? null)

  function snapshot() {
    undoStack.value = [...undoStack.value.slice(-(HISTORY_LIMIT - 1)), cloneElements(elements.value)]
    redoStack.value = []
  }

  function captureHistory() {
    snapshot()
  }

  function setTemplate(template: HudTemplate) {
    snapshot()
    templateName.value = template.name
    elements.value = cloneElements(template.elements)
    selectedId.value = null
    save()
  }

  function addElement(type: HudElementType) {
    if (locked.value) return
    snapshot()
    const element = createElement(type, 44, 44, currentColor.value)
    elements.value.push(element)
    selectedId.value = element.id
    save()
  }

  function addElementAt(type: HudElementType, x: number, y: number, color = currentColor.value) {
    if (locked.value) return null
    snapshot()
    const element = createElement(type, x, y, color)
    elements.value.push(element)
    selectedId.value = element.id
    save()
    return element
  }

  function updateElement(id: string, patch: Partial<HudElement>, recordHistory = true) {
    if (locked.value) return
    const index = elements.value.findIndex((item) => item.id === id)
    if (index < 0) return
    if (recordHistory) snapshot()
    elements.value[index] = { ...elements.value[index], ...patch }
    save()
  }

  function deleteSelected() {
    if (locked.value || !selectedId.value) return
    snapshot()
    elements.value = elements.value.filter((item) => item.id !== selectedId.value)
    selectedId.value = null
    save()
  }

  function undo() {
    const previous = undoStack.value.at(-1)
    if (!previous) return
    redoStack.value = [...redoStack.value, cloneElements(elements.value)]
    undoStack.value = undoStack.value.slice(0, -1)
    elements.value = cloneElements(previous)
    selectedId.value = null
    save()
  }

  function redo() {
    const next = redoStack.value.at(-1)
    if (!next) return
    undoStack.value = [...undoStack.value, cloneElements(elements.value)]
    redoStack.value = redoStack.value.slice(0, -1)
    elements.value = cloneElements(next)
    selectedId.value = null
    save()
  }

  function save() {
    writeToStorage(STORAGE_KEY, {
      name: templateName.value,
      elements: elements.value,
    })
  }

  function restore() {
    const saved = readFromStorage<HudTemplate>(STORAGE_KEY)
    if (!saved?.elements?.length) return
    templateName.value = saved.name || defaultHeroTemplate.name
    const migrated = separateLeftRail(cloneElements(saved.elements))
    elements.value = migrated.elements
    if (migrated.changed) save()
  }

  return {
    editMode,
    locked,
    activeTool,
    currentColor,
    selectedId,
    selectedElement,
    templateName,
    elements,
    defaultHeroTemplate,
    minimalTemplate,
    addElement,
    addElementAt,
    updateElement,
    deleteSelected,
    undo,
    redo,
    save,
    restore,
    setTemplate,
    captureHistory,
  }
})
