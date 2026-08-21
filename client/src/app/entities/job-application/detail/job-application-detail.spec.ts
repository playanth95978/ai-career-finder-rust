import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideRouter, withComponentInputBinding } from '@angular/router';
import { RouterTestingHarness } from '@angular/router/testing';

import { FaIconLibrary } from '@fortawesome/angular-fontawesome';
import { faArrowLeft, faPencilAlt } from '@fortawesome/free-solid-svg-icons';
import { provideTranslateService } from '@ngx-translate/core';
import { of } from 'rxjs';

import { DataUtils } from 'app/core/util/data-util.service';

import { JobApplicationDetail } from './job-application-detail';

describe('JobApplication Management Detail Component', () => {
  let comp: JobApplicationDetail;
  let fixture: ComponentFixture<JobApplicationDetail>;
  let dataUtils: DataUtils;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideRouter(
          [
            {
              path: '**',
              loadComponent: () => import('./job-application-detail').then(m => m.JobApplicationDetail),
              resolve: { jobApplication: () => of({ id: 20361 }) },
            },
          ],
          withComponentInputBinding(),
        ),
      ],
    });
    const library = TestBed.inject(FaIconLibrary);
    library.addIcons(faArrowLeft);
    library.addIcons(faPencilAlt);
    dataUtils = TestBed.inject(DataUtils);
    vitest.spyOn(window, 'open').mockImplementation(() => null);
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(JobApplicationDetail);
    comp = fixture.componentInstance;
  });

  describe('OnInit', () => {
    it('should load jobApplication on init', async () => {
      const harness = await RouterTestingHarness.create();
      const instance = await harness.navigateByUrl('/', JobApplicationDetail);

      // THEN
      expect(instance.jobApplication()).toEqual(expect.objectContaining({ id: 20361 }));
    });
  });

  describe('PreviousState', () => {
    it('should navigate to previous state', () => {
      vitest.spyOn(globalThis.history, 'back');
      comp.previousState();
      expect(globalThis.history.back).toHaveBeenCalled();
    });
  });

  describe('byteSize', () => {
    it('should call byteSize from DataUtils', () => {
      // GIVEN
      vitest.spyOn(dataUtils, 'byteSize');
      const fakeBase64 = 'fake base64';

      // WHEN
      comp.byteSize(fakeBase64);

      // THEN
      expect(dataUtils.byteSize).toHaveBeenCalledWith(fakeBase64);
    });
  });

  describe('openFile', () => {
    it('should call openFile from DataUtils', () => {
      const newWindow = { ...window };
      vitest.stubGlobal(
        'open',
        vitest.fn(() => newWindow),
      );
      window.onload = vitest.fn(() => newWindow) as any;
      window.URL.createObjectURL = vitest.fn() as any;
      // GIVEN
      vitest.spyOn(dataUtils, 'openFile');
      const fakeContentType = 'fake content type';
      const fakeBase64 = 'fake base64';

      // WHEN
      comp.openFile(fakeBase64, fakeContentType);

      // THEN
      expect(dataUtils.openFile).toHaveBeenCalledWith(fakeBase64, fakeContentType);
    });
  });
});
