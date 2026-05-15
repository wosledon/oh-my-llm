import {
  FluentProvider,
  webLightTheme,
  webDarkTheme,
  makeStyles,
  shorthands,
  tokens,
  NavDrawer,
  NavDrawerBody,
  NavDrawerHeader,
  NavItem,
  NavSectionHeader,
  Text,
} from '@fluentui/react-components';
import {
  Home24Regular,
  Server24Regular,
  Cube24Regular,
  Settings24Regular,
  History24Regular,
  DataUsage24Regular,
  Circle16Regular,
} from '@fluentui/react-icons';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { useI18n } from '../i18n';

const useStyles = makeStyles({
  root: {
    display: 'flex',
    height: '100vh',
    width: '100vw',
    overflow: 'hidden',
    backgroundColor: tokens.colorNeutralBackground3,
  },
  navDrawer: {
    width: '260px',
    flexShrink: 0,
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.borderRight('1px', 'solid', tokens.colorNeutralStroke3),
  },
  navHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    ...shorthands.padding('20px', '16px', '12px'),
  },
  navHeaderIcon: {
    color: tokens.colorBrandForeground1,
  },
  navBody: {
    ...shorthands.padding('4px', '12px'),
  },
  content: {
    flex: 1,
    overflow: 'auto',
    ...shorthands.padding('28px', '32px'),
    backgroundColor: tokens.colorNeutralBackground3,
  },
});

export default function Shell() {
  const styles = useStyles();
  const location = useLocation();
  const navigate = useNavigate();
  const [isDark, setIsDark] = useState(false);
  const { t } = useI18n();

  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    setIsDark(media.matches);
    const handler = (e: MediaQueryListEvent) => setIsDark(e.matches);
    media.addEventListener('change', handler);
    return () => media.removeEventListener('change', handler);
  }, []);

  const navItems = [
    { value: '/', label: t.nav.dashboard, icon: Home24Regular },
    { value: '/usage', label: t.nav.usage, icon: DataUsage24Regular },
    { value: '/providers', label: t.nav.providers, icon: Server24Regular },
    { value: '/models', label: t.nav.models, icon: Cube24Regular },
    { value: '/settings', label: t.nav.settings, icon: Settings24Regular },
    { value: '/logs', label: t.nav.logs, icon: History24Regular },
  ];

  return (
    <FluentProvider theme={isDark ? webDarkTheme : webLightTheme}>
      <div className={styles.root}>
        <NavDrawer className={styles.navDrawer} type="inline" open={true}>
          <NavDrawerHeader>
            <div className={styles.navHeader}>
              <Circle16Regular className={styles.navHeaderIcon} />
              <Text weight="semibold" size={400}>
                {t.app.name}
              </Text>
            </div>
          </NavDrawerHeader>
          <NavDrawerBody className={styles.navBody}>
            <NavSectionHeader>{t.app.platform}</NavSectionHeader>
            {navItems.map((item) => (
              <NavItem
                key={item.value}
                icon={<item.icon />}
                value={item.value}
                onClick={() => navigate(item.value)}
                style={
                  location.pathname === item.value
                    ? { backgroundColor: tokens.colorNeutralBackground2 }
                    : undefined
                }
              >
                {item.label}
              </NavItem>
            ))}
          </NavDrawerBody>
        </NavDrawer>
        <div className={styles.content}>
          <Outlet />
        </div>
      </div>
    </FluentProvider>
  );
}
