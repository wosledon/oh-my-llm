import {
  FluentProvider,
  webLightTheme,
  webDarkTheme,
  makeStyles,
  shorthands,
  tokens,
  TabList,
  Tab,
} from '@fluentui/react-components';
import {
  Home24Regular,
  Server24Regular,
  Cube24Regular,
  Settings24Regular,
  History24Regular,
  DataUsage24Regular,
} from '@fluentui/react-icons';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useEffect, useState } from 'react';

const useStyles = makeStyles({
  root: {
    display: 'flex',
    height: '100vh',
    width: '100vw',
    overflow: 'hidden',
    backgroundColor: tokens.colorNeutralBackground3,
  },
  sidebar: {
    width: '220px',
    flexShrink: 0,
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.padding('16px', '8px'),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRight('1px', 'solid', tokens.colorNeutralStroke3),
  },
  content: {
    flex: 1,
    overflow: 'auto',
    ...shorthands.padding('20px'),
  },
});

const navItems = [
  { value: '/', label: 'Dashboard', icon: Home24Regular },
  { value: '/usage', label: 'Usage', icon: DataUsage24Regular },
  { value: '/providers', label: 'Providers', icon: Server24Regular },
  { value: '/models', label: 'Models', icon: Cube24Regular },
  { value: '/settings', label: 'Settings', icon: Settings24Regular },
  { value: '/logs', label: 'Logs', icon: History24Regular },
];

export default function Shell() {
  const styles = useStyles();
  const location = useLocation();
  const navigate = useNavigate();
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    setIsDark(media.matches);
    const handler = (e: MediaQueryListEvent) => setIsDark(e.matches);
    media.addEventListener('change', handler);
    return () => media.removeEventListener('change', handler);
  }, []);

  return (
    <FluentProvider theme={isDark ? webDarkTheme : webLightTheme}>
      <div className={styles.root}>
        <div className={styles.sidebar}>
          <TabList
            vertical
            selectedValue={location.pathname}
            onTabSelect={(_, data) => navigate(String(data.value))}
          >
            {navItems.map((item) => (
              <Tab key={item.value} value={item.value} icon={<item.icon />}>
                {item.label}
              </Tab>
            ))}
          </TabList>
        </div>
        <div className={styles.content}>
          <Outlet />
        </div>
      </div>
    </FluentProvider>
  );
}
