import type { ViewName } from '../types/index';

export function createNavigationStore(initialView: ViewName = 'test') {
  let view = $state<ViewName>(initialView);

  return {
    get view() {
      return view;
    },
    navigate(nextView: ViewName) {
      view = nextView;
    },
  };
}
