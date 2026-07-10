import { useEffect, useRef, type ReactNode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import {
  GoldenLayout,
  type ComponentContainer,
  type LayoutConfig,
} from 'golden-layout';

import CueList from '../components/CueList';
import DetailPanel from '../components/DetailPanel';
import MediaPanel from '../components/MediaPanel';

/// Mount a React node into a Golden Layout component container and unmount it
/// when the tile is destroyed.
function registerComponent(
  layout: GoldenLayout,
  name: string,
  node: ReactNode,
) {
  layout.registerComponentFactoryFunction(
    name,
    (container: ComponentContainer) => {
      const root: Root = createRoot(container.element);
      root.render(node);
      container.on('destroy', () => root.unmount());
    },
  );
}

export default function Workspace() {
  const hostRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const layout = new GoldenLayout(host);

    registerComponent(layout, 'cuelist', <CueList />);
    registerComponent(layout, 'detail', <DetailPanel />);
    registerComponent(layout, 'media', <MediaPanel />);

    const config: LayoutConfig = {
      root: {
        type: 'row',
        content: [
          {
            type: 'column',
            size: '70%',
            content: [
              {
                type: 'component',
                componentType: 'cuelist',
                title: 'Cuelist',
                size: '66.6%',
              },
              {
                type: 'component',
                componentType: 'detail',
                title: 'Detail',
                size: '33.4%',
              },
            ],
          },
          {
            type: 'component',
            componentType: 'media',
            title: 'Active Media',
            size: '30%',
          },
        ],
      },
    };

    layout.loadLayout(config);

    // Keep the layout sized to its (flex) container.
    const resize = () => layout.setSize(host.clientWidth, host.clientHeight);
    const observer = new ResizeObserver(resize);
    observer.observe(host);
    resize();

    return () => {
      observer.disconnect();
      layout.destroy();
    };
  }, []);

  return <div ref={hostRef} className="gl-host" />;
}
