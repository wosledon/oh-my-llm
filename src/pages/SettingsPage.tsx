import { useEffect, useState } from 'react';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import CircularProgress from '@mui/material/CircularProgress';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';

import { useProxyStore } from '../stores/proxyStore';
import { useI18n } from '../i18n';

export default function SettingsPage() {
  const { t } = useI18n();
  const { config, loading, error, fetchConfig, updateConfig } = useProxyStore();

  const [autoStart, setAutoStart] = useState(false);
  const [logRequests, setLogRequests] = useState(true);
  const [logDays, setLogDays] = useState('30');
  const [maxRetries, setMaxRetries] = useState('3');
  const [timeout, setTimeout] = useState('120');

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  useEffect(() => {
    if (config) {
      setAutoStart(config.auto_start);
      setLogRequests(config.log_requests);
      setLogDays(String(config.log_retention_days));
      setMaxRetries(String(config.max_retries));
      setTimeout(String(config.timeout_secs));
    }
  }, [config]);

  const handleUpdate = (patch: Partial<typeof config>) => {
    if (!config) return;
    updateConfig({ ...config, ...patch });
  };

  if (!config) return <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />;

  return (
    <Box>
      <Typography variant="h4" sx={{ fontWeight: 700, mb: 3, pb: 2, borderBottom: '1px solid', borderColor: 'divider', letterSpacing: -0.5 }}>
        {t.settings.title}
      </Typography>

      {loading && <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />}
      {error && (
        <Alert severity="error" sx={{ mb: 2, borderRadius: 2 }}>{t.common.error}: {error}</Alert>
      )}

      <Box
        sx={{
          display: 'grid',
          gap: 2.5,
          gridTemplateColumns: { xs: '1fr', md: '1fr 1fr', lg: '1fr 1fr 1fr' },
        }}
      >
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {t.settings.autoStart}
            </Typography>
            <FormControlLabel
              control={
                <Switch
                  checked={autoStart}
                  onChange={(e) => {
                    setAutoStart(e.target.checked);
                    handleUpdate({ auto_start: e.target.checked });
                  }}
                />
              }
              label={
                <Typography variant="body2" color="text.secondary">
                  {autoStart ? t.dashboard.active : t.dashboard.inactive}
                </Typography>
              }
            />
          </CardContent>
        </Card>

        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {t.settings.logRequests}
            </Typography>
            <FormControlLabel
              control={
                <Switch
                  checked={logRequests}
                  onChange={(e) => {
                    setLogRequests(e.target.checked);
                    handleUpdate({ log_requests: e.target.checked });
                  }}
                />
              }
              label={
                <Typography variant="body2" color="text.secondary">
                  {logRequests ? t.dashboard.active : t.dashboard.inactive}
                </Typography>
              }
            />
            <TextField
              size="small"
              label={t.settings.logRetentionDays}
              type="number"
              value={logDays}
              onChange={(e) => setLogDays(e.target.value)}
              onBlur={() => handleUpdate({ log_retention_days: parseInt(logDays) || 30 })}
              fullWidth
            />
          </CardContent>
        </Card>

        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {t.settings.maxRetries}
            </Typography>
            <TextField
              size="small"
              type="number"
              value={maxRetries}
              onChange={(e) => setMaxRetries(e.target.value)}
              onBlur={() => handleUpdate({ max_retries: parseInt(maxRetries) || 3 })}
              fullWidth
            />
          </CardContent>
        </Card>

        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {t.settings.timeoutSecs}
            </Typography>
            <TextField
              size="small"
              type="number"
              value={timeout}
              onChange={(e) => setTimeout(e.target.value)}
              onBlur={() => handleUpdate({ timeout_secs: parseInt(timeout) || 120 })}
              fullWidth
            />
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
}
