import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { IAutoApplyConfig } from '../auto-apply-config.model';
import { AutoApplyConfigService } from '../service/auto-apply-config.service';

import { AutoApplyConfigFormService } from './auto-apply-config-form.service';
import { AutoApplyConfigUpdate } from './auto-apply-config-update';

describe('AutoApplyConfig Management Update Component', () => {
  let comp: AutoApplyConfigUpdate;
  let fixture: ComponentFixture<AutoApplyConfigUpdate>;
  let activatedRoute: ActivatedRoute;
  let autoApplyConfigFormService: AutoApplyConfigFormService;
  let autoApplyConfigService: AutoApplyConfigService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(AutoApplyConfigUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    autoApplyConfigFormService = TestBed.inject(AutoApplyConfigFormService);
    autoApplyConfigService = TestBed.inject(AutoApplyConfigService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should update editForm', () => {
      const autoApplyConfig: IAutoApplyConfig = { id: 29750 };

      activatedRoute.data = of({ autoApplyConfig });
      comp.ngOnInit();

      expect(comp.autoApplyConfig).toEqual(autoApplyConfig);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IAutoApplyConfig>();
      const autoApplyConfig = { id: 30992 };
      vitest.spyOn(autoApplyConfigFormService, 'getAutoApplyConfig').mockReturnValue(autoApplyConfig);
      vitest.spyOn(autoApplyConfigService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ autoApplyConfig });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(autoApplyConfig);
      saveSubject.complete();

      // THEN
      expect(autoApplyConfigFormService.getAutoApplyConfig).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(autoApplyConfigService.update).toHaveBeenCalledWith(expect.objectContaining(autoApplyConfig));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IAutoApplyConfig>();
      const autoApplyConfig = { id: 30992 };
      vitest.spyOn(autoApplyConfigFormService, 'getAutoApplyConfig').mockReturnValue({ id: null });
      vitest.spyOn(autoApplyConfigService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ autoApplyConfig: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(autoApplyConfig);
      saveSubject.complete();

      // THEN
      expect(autoApplyConfigFormService.getAutoApplyConfig).toHaveBeenCalled();
      expect(autoApplyConfigService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IAutoApplyConfig>();
      const autoApplyConfig = { id: 30992 };
      vitest.spyOn(autoApplyConfigService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ autoApplyConfig });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(autoApplyConfigService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });
});
