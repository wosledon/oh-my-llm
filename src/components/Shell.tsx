import Box from '@mui/material/Box';
import Drawer from '@mui/material/Drawer';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';
import Divider from '@mui/material/Divider';
import HomeIcon from '@mui/icons-material/Home';
import DnsIcon from '@mui/icons-material/Dns';
import ViewInArIcon from '@mui/icons-material/ViewInAr';
import SettingsIcon from '@mui/icons-material/Settings';
import HistoryIcon from '@mui/icons-material/History';
import BarChartIcon from '@mui/icons-material/BarChart';
import HubIcon from '@mui/icons-material/Hub';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import { useI18n } from '../i18n';

const drawerWidth = 260;

export default function Shell() {
  const location = useLocation();
  const navigate = useNavigate();
  const { t } = useI18n();

  const navItems = [
    { value: '/', label: t.nav.dashboard, icon: HomeIcon },
    { value: '/usage', label: t.nav.usage, icon: BarChartIcon },
    { value: '/providers', label: t.nav.providers, icon: DnsIcon },
    { value: '/models', label: t.nav.models, icon: ViewInArIcon },
    { value: '/settings', label: t.nav.settings, icon: SettingsIcon },
    { value: '/logs', label: t.nav.logs, icon: HistoryIcon },
  ];

  return (
    <Box sx={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <Drawer
        variant="permanent"
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
            bgcolor: 'background.paper',
            borderRight: '1px solid',
            borderColor: 'divider',
          },
        }}
      >
        <Toolbar sx={{ display: 'flex', alignItems: 'center', gap: 1.5, px: 2 }}>
          <HubIcon sx={{ color: 'primary.main', fontSize: 24 }} />
          <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>
            {t.app.name}
          </Typography>
        </Toolbar>
        <Divider />
        <List sx={{ px: 1.5, py: 1 }}>
          <ListItem sx={{ py: 0.5 }}>
            <Typography variant="caption" color="text.secondary" sx={{ px: 1, fontWeight: 500, textTransform: 'uppercase', letterSpacing: 0.5 }}>
              {t.app.platform}
            </Typography>
          </ListItem>
          {navItems.map((item) => (
            <ListItem key={item.value} disablePadding sx={{ mb: 0.5 }}>
              <ListItemButton
                selected={location.pathname === item.value}
                onClick={() => navigate(item.value)}
                sx={{ borderRadius: 1.5 }}
              >
                <ListItemIcon sx={{ minWidth: 36 }}>
                  <item.icon fontSize="small" />
                </ListItemIcon>
                <ListItemText
                  primary={item.label}
                  sx={{
                    '& .MuiListItemText-primary': {
                      fontSize: 14,
                      fontWeight: location.pathname === item.value ? 600 : 400,
                    },
                  }}
                />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Drawer>
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          bgcolor: 'background.default',
          overflow: 'auto',
          p: 4,
        }}
      >
        <Outlet />
      </Box>
    </Box>
  );
}
