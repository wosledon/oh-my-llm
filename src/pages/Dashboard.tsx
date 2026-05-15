import { useEffect } from 'react';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import { useProxyStore } from '../stores/proxyStore';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';
import { useI18n } from '../i18n';

export default function Dashboard() {
  const { t } = useI18n();
  const { config, running, getStatus, startProxy, stopProxy } = useProxyStore();
  const { providers, fetchProviders } = useProviderStore();
  const { models, fetchModels } = useModelStore();

  useEffect(() => {
    getStatus();
    fetchProviders();
    fetchModels();
  }, [getStatus, fetchProviders, fetchModels]);

  return (
    <Box>
      <Typography variant="h4" sx={{ fontWeight: 600, mb: 3 }}>
        {t.dashboard.title}
      </Typography>

      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
          gap: 3,
        }}
      >
        <Card variant="outlined">
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.proxyStatus}
              </Typography>
              <Chip
                size="small"
                color={running ? 'success' : 'error'}
                label={running ? t.dashboard.running : t.dashboard.stopped}
              />
            </Box>
            <Typography variant="h3" sx={{ fontWeight: 700, mt: 2 }}>
              {config ? `${config.port}` : '--'}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.port}
            </Typography>
            <Box sx={{ mt: 2 }}>
              <Button
                variant="contained"
                size="small"
                startIcon={running ? <StopIcon /> : <PlayArrowIcon />}
                color={running ? 'inherit' : 'primary'}
                onClick={running ? stopProxy : startProxy}
              >
                {running ? t.dashboard.stop : t.dashboard.start}
              </Button>
            </Box>
          </CardContent>
        </Card>

        <Card variant="outlined">
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.providers}
              </Typography>
              <Chip size="small" variant="outlined" label={providers.length} />
            </Box>
            <Typography variant="h3" sx={{ fontWeight: 700, mt: 2 }}>
              {providers.length}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.total}
            </Typography>
          </CardContent>
        </Card>

        <Card variant="outlined">
          <CardContent>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <Typography color="text.secondary" sx={{ fontWeight: 500 }}>
                {t.dashboard.models}
              </Typography>
              <Chip size="small" variant="outlined" label={models.length} />
            </Box>
            <Typography variant="h3" sx={{ fontWeight: 700, mt: 2 }}>
              {models.length}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.total}</Typography>
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
}
