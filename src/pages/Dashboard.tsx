import { useEffect, useState } from 'react';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import ListSubheader from '@mui/material/ListSubheader';
import Divider from '@mui/material/Divider';
import { alpha } from '@mui/material/styles';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import CheckIcon from '@mui/icons-material/Check';
import OpenInNewIcon from '@mui/icons-material/OpenInNew';
import DnsIcon from '@mui/icons-material/Dns';
import ViewInArIcon from '@mui/icons-material/ViewInAr';
import SettingsIcon from '@mui/icons-material/Settings';
import { useProxyStore } from '../stores/proxyStore';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';
import { useI18n } from '../i18n';

export default function Dashboard() {
  const { t } = useI18n();
  const { config, running, getStatus, startProxy, stopProxy, updateConfig } = useProxyStore();
  const { providers, fetchProviders } = useProviderStore();
  const { models, fetchModels } = useModelStore();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    getStatus();
    fetchProviders();
    fetchModels();
  }, [getStatus, fetchProviders, fetchModels]);

  const proxyAddress = config ? `http://127.0.0.1:${config.port}/v1` : '';

  const handleCopy = async () => {
    if (!proxyAddress) return;
    try {
      await navigator.clipboard.writeText(proxyAddress);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // ignore
    }
  };

  const enabledModels = models.filter((m) => m.enabled).length;

  const iconBg = (paletteKey: 'success' | 'error' | 'primary' | 'info' | 'warning' | 'secondary') => ({
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    width: 36,
    height: 36,
    borderRadius: 1.5,
    bgcolor: (theme: any) => alpha(theme.palette[paletteKey].main, 0.12),
    color: (theme: any) => theme.palette[paletteKey].main,
  });

  return (
    <Box>
      <Typography variant="h4" sx={{ fontWeight: 700, mb: 3, letterSpacing: -0.5 }}>
        {t.dashboard.title}
      </Typography>

      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, 1fr)', lg: 'repeat(3, 1fr)' },
          gap: 2.5,
        }}
      >
        {/* Proxy Status Card */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 2.5 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Box sx={iconBg(running ? 'success' : 'error')}>
                {running ? <StopIcon fontSize="small" /> : <PlayArrowIcon fontSize="small" />}
              </Box>
              <Chip
                size="small"
                color={running ? 'success' : 'error'}
                label={running ? t.dashboard.running : t.dashboard.stopped}
                sx={{ fontWeight: 600 }}
              />
            </Box>
            <Typography variant="h4" sx={{ fontWeight: 800, lineHeight: 1.2 }}>
              {config ? `${config.port}` : '--'}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.port}
            </Typography>
            <Box sx={{ mt: 2 }}>
              <Button
                variant={running ? 'outlined' : 'contained'}
                size="small"
                startIcon={running ? <StopIcon /> : <PlayArrowIcon />}
                color={running ? 'error' : 'primary'}
                onClick={running ? stopProxy : startProxy}
                fullWidth
              >
                {running ? t.dashboard.stop : t.dashboard.start}
              </Button>
            </Box>
          </CardContent>
        </Card>

        {/* Proxy Address Card */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 2.5 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Box sx={iconBg('primary')}>
                <OpenInNewIcon fontSize="small" />
              </Box>
            </Box>
            <Typography
              variant="body1"
              sx={{ fontWeight: 600, fontFamily: 'monospace', wordBreak: 'break-all', lineHeight: 1.4 }}
            >
              {proxyAddress || '--'}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.proxyAddress}
            </Typography>
            <Box sx={{ mt: 2 }}>
              <Button
                variant="outlined"
                size="small"
                startIcon={copied ? <CheckIcon /> : <ContentCopyIcon />}
                onClick={handleCopy}
                disabled={!proxyAddress}
                fullWidth
              >
                {copied ? t.dashboard.copied : t.dashboard.copyAddress}
              </Button>
            </Box>
          </CardContent>
        </Card>

        {/* Shadow Model Card */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 2.5 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Box sx={iconBg('info')}>
                <ViewInArIcon fontSize="small" />
              </Box>
            </Box>
            <Typography variant="h5" sx={{ fontWeight: 700, fontFamily: 'monospace', lineHeight: 1.3 }}>
              {config?.shadow_model_name || '--'}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.shadowModel}
            </Typography>
            <Box sx={{ mt: 2 }}>
              <FormControlLabel
                control={
                  <Switch
                    size="small"
                    checked={!!config?.shadow_mapping_id}
                    onChange={async (e) => {
                      if (!config) return;
                      const shadow_mapping_id = e.target.checked
                        ? (config.shadow_mapping_id || models[0]?.id || '')
                        : null;
                      await updateConfig({ ...config, shadow_mapping_id });
                    }}
                    disabled={models.length === 0}
                  />
                }
                label={
                  <Typography variant="body2">
                    {t.dashboard.shadowModelSwitch}
                  </Typography>
                }
              />
              {!!config?.shadow_mapping_id && (
                <FormControl fullWidth size="small" sx={{ mt: 1 }}>
                  <InputLabel>{t.dashboard.shadowModelSelectLabel}</InputLabel>
                  <Select
                    value={config.shadow_mapping_id}
                    label={t.dashboard.shadowModelSelectLabel}
                    onChange={async (e) => {
                      if (!config) return;
                      await updateConfig({ ...config, shadow_mapping_id: e.target.value });
                    }}
                    MenuProps={{
                      slotProps: {
                        paper: {
                          sx: {
                            mt: 1,
                            borderRadius: 2,
                            boxShadow: (theme) => theme.shadows[8],
                            minWidth: 260,
                          },
                        },
                      },
                    }}
                  >
                    {providers
                      .filter((p) => models.some((m) => m.provider_id === p.id))
                      .map((provider, pIdx, filteredProviders) => {
                        const providerModels = models.filter((m) => m.provider_id === provider.id);
                        return [
                          <ListSubheader key={`sub-${provider.id}`} sx={{ fontWeight: 700, bgcolor: 'background.paper' }}>
                            {provider.name}
                          </ListSubheader>,
                          ...providerModels.map((m) => (
                            <MenuItem key={m.id} value={m.id} sx={{ pl: 3 }}>
                              {m.upstream_name}
                            </MenuItem>
                          )),
                          pIdx < filteredProviders.length - 1 && (
                            <Divider key={`div-${provider.id}`} component="li" />
                          ),
                        ];
                      })}
                  </Select>
                </FormControl>
              )}
            </Box>
          </CardContent>
        </Card>

        {/* Providers Card */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 2.5 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Box sx={iconBg('warning')}>
                <DnsIcon fontSize="small" />
              </Box>
            </Box>
            <Typography variant="h4" sx={{ fontWeight: 800, lineHeight: 1.2 }}>
              {providers.length}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {t.dashboard.total}
            </Typography>
          </CardContent>
        </Card>

        {/* Models Card */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 2.5 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Box sx={iconBg('secondary')}>
                <ViewInArIcon fontSize="small" />
              </Box>
            </Box>
            <Typography variant="h4" sx={{ fontWeight: 800, lineHeight: 1.2 }}>
              {models.length}
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
              {enabledModels}{t.dashboard.enabledCount}
            </Typography>
          </CardContent>
        </Card>

        {/* Protocols Card */}
        <Card sx={{ borderRadius: 3, border: '1px solid', borderColor: 'divider' }}>
          <CardContent sx={{ p: 2.5 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Box sx={iconBg('success')}>
                <SettingsIcon fontSize="small" />
              </Box>
            </Box>
            <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', mb: 1 }}>
              <Chip
                size="small"
                variant={config?.openai_enabled ? 'filled' : 'outlined'}
                color={config?.openai_enabled ? 'success' : 'default'}
                label="OpenAI"
                sx={{ fontWeight: 600 }}
              />
              <Chip
                size="small"
                variant={config?.anthropic_enabled ? 'filled' : 'outlined'}
                color={config?.anthropic_enabled ? 'success' : 'default'}
                label="Anthropic"
                sx={{ fontWeight: 600 }}
              />
            </Box>
            <Typography variant="body2" color="text.secondary">
              {config?.openai_enabled || config?.anthropic_enabled
                ? t.dashboard.active
                : t.dashboard.inactive}
            </Typography>
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
}
